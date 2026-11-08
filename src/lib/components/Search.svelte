<script lang="ts">
  import { api, errorMessage, type SearchHit } from "../api";
  import { workspace } from "../state/workspace.svelte";
  import { debounce } from "../utils/debounce";

  export const SEARCH_DEBOUNCE_MS = 200;

  let query = $state("");
  let hits = $state<SearchHit[]>([]);
  let error = $state<string | null>(null);
  let input: HTMLInputElement | undefined = $state();
  let open = $state(false);

  const run = debounce(async (q: string) => {
    if (!q.trim()) {
      hits = [];
      return;
    }
    try {
      hits = await api.search(q, 50);
      error = null;
    } catch (err) {
      error = errorMessage(err);
      hits = [];
    }
  }, SEARCH_DEBOUNCE_MS);

  function onInput() {
    open = true;
    run(query);
  }

  async function choose(hit: SearchHit) {
    if (workspace.selectedFolderId !== null && workspace.selectedFolderId !== hit.folder_id) {
      await workspace.selectFolder(hit.folder_id);
    }
    await workspace.selectNote(hit.id);
    open = false;
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Enter" && hits.length) {
      event.preventDefault();
      choose(hits[0]);
    } else if (event.key === "Escape") {
      open = false;
      query = "";
      hits = [];
      input?.blur();
    }
  }

  export function focus(): void {
    input?.focus();
    input?.select();
    open = true;
  }

  function parts(snippet: string): { text: string; mark: boolean }[] {
    return snippet
      .split(/(\[[^\]]*\])/)
      .filter((s) => s.length)
      .map((s) =>
        s.startsWith("[") && s.endsWith("]") ? { text: s.slice(1, -1), mark: true } : { text: s, mark: false },
      );
  }
</script>

<div class="search">
  <input
    type="search"
    placeholder="Search (⌘/Ctrl+K)"
    bind:value={query}
    bind:this={input}
    oninput={onInput}
    onkeydown={onKey}
    onfocus={() => (open = true)}
    aria-label="Search notes"
    autocomplete="off"
    spellcheck="false"
  />
  {#if open && query.trim()}
    <div class="results" role="listbox">
      {#if error}
        <p class="empty">{error}</p>
      {:else if hits.length === 0}
        <p class="empty">No matches</p>
      {:else}
        {#each hits as hit (hit.id)}
          <button type="button" role="option" aria-selected="false" onclick={() => choose(hit)}>
            <span class="hit-title">{hit.title}</span>
            <span class="snippet">
              {#each parts(hit.snippet) as part, i (i)}
                {#if part.mark}<mark>{part.text}</mark>{:else}{part.text}{/if}
              {/each}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .search {
    position: relative;
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  input {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
  }
  .results {
    position: absolute;
    left: 8px;
    right: 8px;
    top: 100%;
    z-index: 10;
    max-height: 60vh;
    overflow-y: auto;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    padding: 4px;
  }
  .results button {
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
  .results button:hover {
    background: var(--bg-muted);
  }
  .hit-title {
    font-weight: 600;
  }
  .snippet {
    font-size: 12px;
    color: var(--fg-muted);
  }
  mark {
    background: color-mix(in srgb, var(--accent) 25%, transparent);
    color: inherit;
    border-radius: 2px;
  }
  .empty {
    margin: 0;
    padding: 8px 10px;
    color: var(--fg-muted);
    font-size: 12px;
  }
</style>
